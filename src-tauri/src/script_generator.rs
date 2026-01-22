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
        var regexPattern = ex
            .replace(/\./g, '\\.')
            .replace(/\\\.\*/g, '.*');
        
        var regex = new RegExp(regexPattern);
        return regex.test(text);
    }} catch (e) {{
        log("Invalid regex pattern '" + ex + "': " + e);
        return false;
    }}
}}

// 枚举应用自己的类（借鉴 r0tracer）
function enumerateAppClasses(callback) {{
    Java.enumerateClassLoaders({{
        onMatch: function(loader) {{
            try {{
                // 查找应用的 ClassLoader
                if (loader.toString().indexOf("PathClassLoader") >= 0) {{
                    Java.classFactory.loader = loader;
                    
                    var BaseDexClassLoader = Java.use("dalvik.system.BaseDexClassLoader");
                    var pathcl = Java.cast(loader, BaseDexClassLoader);
                    var DexPathList = Java.use("dalvik.system.DexPathList");
                    var dexPathList = Java.cast(pathcl.pathList.value, DexPathList);
                    var DexFile = Java.use("dalvik.system.DexFile");
                    var Element = Java.use("dalvik.system.DexPathList$Element");
                    
                    for (var i = 0; i < dexPathList.dexElements.value.length; i++) {{
                        var element = Java.cast(dexPathList.dexElements.value[i], Element);
                        if (element.dexFile.value) {{
                            var dexFile = Java.cast(element.dexFile.value, DexFile);
                            var mcookie = dexFile.mCookie.value;
                            if (dexFile.mInternalCookie.value) {{
                                mcookie = dexFile.mInternalCookie.value;
                            }}
                            
                            try {{
                                var classNames = element.dexFile.value.getClassNameList(mcookie);
                                for (var j = 0; j < classNames.length; j++) {{
                                    callback(classNames[j]);
                                }}
                            }} catch (e) {{
                                // 继续处理下一个
                            }}
                        }}
                    }}
                }}
            }} catch (e) {{
                // 继续处理下一个 ClassLoader
            }}
        }},
        onComplete: function() {{}}
    }});
}}

if (Java.available) {{
    Java.perform(function () {{
        log('M-FTracert Start...');
        
        var matchRegEx = {match_regex};
        var blackRegEx = {black_regex};
        var hookedClasses = {{}};
        
        function tryHookClass(aClass) {{
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
        
        var count = 0;
        
        // 方法1：枚举应用自己的类（更可靠）
        log("Enumerating app classes from DEX...");
        enumerateAppClasses(function(className) {{
            if (tryHookClass(className)) {{
                count++;
            }}
        }});
        
        // 方法2：枚举已加载的类（作为补充）
        log("Enumerating loaded classes...");
        Java.enumerateLoadedClasses({{
            onMatch: function (aClass) {{
                if (tryHookClass(aClass)) {{
                    count++;
                }}
            }},
            onComplete: function () {{
                log("Hooked " + count + " classes total");
                log("M-FTracert ready. Monitoring for method calls...");
            }}
        }});
    }});
}}
"#, match_regex = match_regex, black_regex = black_regex)
}
