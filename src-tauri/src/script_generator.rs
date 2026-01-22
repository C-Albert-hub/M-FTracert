use crate::models::{MatchRule, BlacklistRule};

pub fn generate_frida_script(match_rules: &[MatchRule], blacklist_rules: &[BlacklistRule]) -> String {
    // 为所有规则自动添加 M: 前缀
    let match_patterns: Vec<String> = match_rules.iter()
        .map(|r| {
            if r.pattern.starts_with("M:") {
                r.pattern.clone()
            } else {
                format!("M:{}", r.pattern)
            }
        })
        .collect();
    
    let blacklist_patterns: Vec<String> = blacklist_rules.iter()
        .map(|r| {
            if r.pattern.starts_with("M:") {
                r.pattern.clone()
            } else {
                format!("M:{}", r.pattern)
            }
        })
        .collect();
    
    let match_regex = serde_json::to_string(&match_patterns).unwrap_or_else(|_| "[]".to_string());
    let black_regex = serde_json::to_string(&blacklist_patterns).unwrap_or_else(|_| "[]".to_string());

    format!(r#"
function log(text) {{
    var packet = {{'cmd': 'log','data': text}};
    send("tracer::" + JSON.stringify(packet));
}}

function enter(tid, tname, cls, method, args) {{
    var packet = {{'cmd': 'enter','data': [tid, tname, cls, method, args]}};
    send("tracer::" + JSON.stringify(packet));
}}

function exit(tid, retval) {{
    var packet = {{'cmd': 'exit','data': [tid, retval]}};
    send("tracer::" + JSON.stringify(packet));
}}

function getTid() {{
    var Thread = Java.use("java.lang.Thread");
    return Thread.currentThread().getId();
}}

function getTName() {{
    var Thread = Java.use("java.lang.Thread");
    return Thread.currentThread().getName();
}}

function traceClass(clsname) {{
    try {{
        var target = Java.use(clsname);
        var methods = target.class.getDeclaredMethods();
        var hookedCount = 0;
        methods.forEach(function (method) {{
            var methodName = method.getName();
            // 跳过一些不需要 hook 的方法
            if (methodName.indexOf('access$') === 0 || methodName === 'toString' || methodName === 'hashCode' || methodName === 'equals') {{
                return;
            }}
            
            try {{
                var overloads = target[methodName].overloads;
                overloads.forEach(function (overload) {{
                    try {{
                        var proto = "(";
                        overload.argumentTypes.forEach(function (type) {{
                            proto += type.className + ", ";
                        }});
                        if (proto.length > 1) {{
                            proto = proto.substr(0, proto.length - 2);
                        }}
                        proto += ")";
                        
                        var fullMethodName = clsname + "." + methodName + proto;
                        log("hooking: " + fullMethodName);
                        
                        // 保存原始实现的引用
                        var originalImpl = overload.implementation;
                        
                        overload.implementation = function () {{
                            var args = [];
                            var tid = getTid();
                            var tName = getTName();
                            for (var j = 0; j < arguments.length; j++) {{
                                try {{
                                    args[j] = arguments[j] + "";
                                }} catch (e) {{
                                    args[j] = "[Object]";
                                }}
                            }}
                            enter(tid, tName, clsname, methodName + proto, args);
                            try {{
                                var retval = this[methodName].apply(this, arguments);
                                exit(tid, retval !== null && retval !== undefined ? ("" + retval) : "null");
                                return retval;
                            }} catch (e) {{
                                exit(tid, "[Exception: " + e + "]");
                                throw e;
                            }}
                        }};
                        
                        hookedCount++;
                    }} catch (e) {{
                        // 忽略单个重载的错误
                    }}
                }});
            }} catch (e) {{
                // 忽略单个方法的错误
            }}
        }});
        
        if (hookedCount > 0) {{
            log("Successfully hooked " + hookedCount + " methods in " + clsname);
        }}
    }} catch (e) {{
        log("'" + clsname + "' hook fail: " + e);
    }}
}}

function match(ex, text) {{
    // 支持 M: 前缀（匹配模式）或直接正则表达式
    if (ex[1] == ':') {{
        var mode = ex[0];
        if (mode == 'M') {{
            ex = ex.substr(2, ex.length - 2);
        }} else {{
            log("Unknown match mode: " + mode + ", only M (match) is supported");
            return false;
        }}
    }}
    
    try {{
        // 转义正则表达式中的特殊字符（除了 * 和 .）
        // 将 .* 转换为正则表达式的 .*
        // 将单独的 . 转义为 \.
        var regexPattern = ex
            .replace(/\./g, '\\.')  // 先转义所有点
            .replace(/\\\.\*/g, '.*');  // 然后将 \.\* 还原为 .*
        
        var regex = new RegExp(regexPattern);
        return regex.test(text);
    }} catch (e) {{
        log("Invalid regex pattern '" + ex + "': " + e);
        return false;
    }}
}}

if (Java.available) {{
    Java.perform(function () {{
        log('M-FTracert Start...');
        
        // 禁用 ART 优化，确保 hook 能立即生效
        try {{
            Java.deoptimizeEverything();
            log("ART optimization disabled for better hooking");
        }} catch (e) {{
            log("Failed to disable ART optimization: " + e);
        }}
        
        var matchRegEx = {match_regex};
        var blackRegEx = {black_regex};
        var hookedClasses = {{}};
        
        function tryHookClass(aClass) {{
            // 避免重复 hook
            if (hookedClasses[aClass]) {{
                return false;
            }}
            
            for (var index in matchRegEx) {{
                if (match(matchRegEx[index], aClass)) {{
                    var is_black = false;
                    for (var i in blackRegEx) {{
                        if (match(blackRegEx[i], aClass)) {{
                            is_black = true;
                            log(aClass + "' black by '" + blackRegEx[i] + "'");
                            break;
                        }}
                    }}
                    if (is_black) {{
                        break;
                    }}
                    log(aClass + "' match by '" + matchRegEx[index] + "'");
                    traceClass(aClass);
                    hookedClasses[aClass] = true;
                    return true;
                }}
            }}
            return false;
        }}
        
        function enumerateAndHook() {{
            var count = 0;
            var checked = 0;
            Java.enumerateLoadedClasses({{
                onMatch: function (aClass) {{
                    checked++;
                    if (tryHookClass(aClass)) {{
                        count++;
                    }}
                }},
                onComplete: function () {{
                    log("Checked " + checked + " classes, hooked " + count + " classes in this round");
                }}
            }});
            return count;
        }}
        
        // 第一次枚举
        var initialCount = enumerateAndHook();
        log("Initial enumeration complete. Hooked " + initialCount + " classes.");
        
        // 强制刷新所有已 hook 的类
        try {{
            Java.deoptimizeBootImage();
            log("Boot image deoptimized");
        }} catch (e) {{
            // 忽略错误
        }}
        
        // 延迟重新枚举，捕获延迟加载的类
        setTimeout(function() {{
            log("Re-enumerating classes after 3 seconds...");
            var newCount = enumerateAndHook();
            if (newCount > 0) {{
                log("Found and hooked " + newCount + " new classes");
            }}
        }}, 3000);
        
        // 再次延迟枚举
        setTimeout(function() {{
            log("Re-enumerating classes after 8 seconds...");
            var newCount = enumerateAndHook();
            if (newCount > 0) {{
                log("Found and hooked " + newCount + " new classes");
            }}
            log("All hooks installed and ready!");
        }}, 8000);
        
        // 监听新加载的类（备用方案）
        try {{
            var DexClassLoader = Java.use("dalvik.system.BaseDexClassLoader");
            var originalLoadClass = DexClassLoader.loadClass.overload("java.lang.String");
            originalLoadClass.implementation = function(className) {{
                var result = originalLoadClass.call(this, className);
                // 异步尝试 hook，避免阻塞类加载
                setTimeout(function() {{
                    tryHookClass(className);
                }}, 10);
                return result;
            }};
            log("ClassLoader monitor installed");
        }} catch (e) {{
            log("ClassLoader monitor failed: " + e);
        }}
        
        log("M-FTracert ready. Monitoring for method calls...");
        log("IMPORTANT: Wait at least 10 seconds before testing to ensure all hooks are active!");
    }});
}}
"#, match_regex = match_regex, black_regex = black_regex)
}
