# Wry's JNI binding macro generates symbols looked up via JNI_OnLoad-style
# discovery. R8/ProGuard would otherwise strip them as unused from the host
# APK (no Java caller ever references them — the JVM finds them by name).
-keep class org.godotwry.plugin.** { *; }
-keep class * extends org.godotengine.godot.plugin.GodotPlugin { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}
