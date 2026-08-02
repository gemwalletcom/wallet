# UniFFI registers generated native methods through JNA and JNA reflects over
# callback methods, Structure fields, and @Structure.FieldOrder annotations.
-keep class com.sun.jna.** { *; }
-keep class uniffi.gemstone.** { *; }
-keepattributes RuntimeVisibleAnnotations

# JNA packages desktop-only AWT helpers that are unreachable on Android.
-dontwarn java.awt.Component
-dontwarn java.awt.GraphicsEnvironment
-dontwarn java.awt.HeadlessException
-dontwarn java.awt.Window
