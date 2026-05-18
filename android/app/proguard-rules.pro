# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.kts.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Preserve stack traces and fix R8 non-deterministic map-id for reproducible builds.
-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile

-verbose
#-dontobfuscate
-ignorewarnings

# These lines allow optimisation whilst preserving stack traces
-optimizations !code/allocation/variable
-optimizations !class/unboxing/enum
-keep,allowshrinking,allowoptimization class * { <methods>; }
-keepattributes Signature

-dontwarn com.google.firebase.analytics.connector.AnalyticsConnector

-keep class wallet.core.jni.** { *; }
-keep class wallet.core.jni.proto.** { *; }

-keep class com.gemwallet.android.** { *; }
-keep class com.wallet.core.** { *; }
-keep class com.sun.jna.** { *; }
-keep class * extends com.sun.jna.Structure { *; }
-keepattributes *Annotation*,InnerClasses,EnclosingMethod,Signature
-keep @androidx.annotation.Keep class ** { *; }
-keepclassmembers class ** { @androidx.annotation.Keep *; }
-keep class java.nio.** { *; }
-keep class * implements java.nio.** { *; }
-keep class uniffi.** { *; }
-keep class uniffi.Gemstone.** { *; }

#Retrofit
# Retrofit does reflection on generic parameters. InnerClasses is required to use Signature and
# EnclosingMethod is required to use InnerClasses.
-keepattributes Signature, InnerClasses, EnclosingMethod

# Retrofit does reflection on method and parameter annotations.
-keepattributes RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations

# Keep annotation default values (e.g., retrofit2.http.Field.encoded).
-keepattributes AnnotationDefault

# Retain service method parameters when optimizing.
-keepclassmembers,allowshrinking,allowobfuscation interface * {
    @retrofit2.http.* <methods>;
}

# Ignore annotation used for build tooling.
-dontwarn org.codehaus.mojo.animal_sniffer.IgnoreJRERequirement

# Ignore JSR 305 annotations for embedding nullability information.
-dontwarn javax.annotation.**

# Guarded by a NoClassDefFoundError try/catch and only used when on the classpath.
-dontwarn kotlin.Unit

# Top-level functions that can only be used by Kotlin.
-dontwarn retrofit2.KotlinExtensions
-dontwarn retrofit2.KotlinExtensions$*

# With R8 full mode, it sees no subtypes of Retrofit interfaces since they are created with a Proxy
# and replaces all potential values with null. Explicitly keeping the interfaces prevents this.
-if interface * { @retrofit2.http.* <methods>; }
-keep,allowobfuscation interface <1>

# Keep inherited services.
-if interface * { @retrofit2.http.* <methods>; }
-keep,allowobfuscation interface * extends <1>

# With R8 full mode generic signatures are stripped for classes that are not
# kept. Suspend functions are wrapped in continuations where the type argument
# is used.
-keep,allowobfuscation,allowshrinking class kotlin.coroutines.Continuation

# R8 full mode strips generic signatures from return types if not kept.
-if interface * { @retrofit2.http.* public *** *(...); }
-keep,allowoptimization,allowshrinking,allowobfuscation class <3>

# JSR 305 annotations are for embedding nullability information.
-dontwarn javax.annotation.**

# A resource is loaded with a relative path so the package of this class must be preserved.
-adaptresourcefilenames okhttp3/internal/publicsuffix/PublicSuffixDatabase.gz

# Animal Sniffer compileOnly dependency to ensure APIs are compatible with older versions of Java.
-dontwarn org.codehaus.mojo.animal_sniffer.*

-keepnames @dagger.hilt.android.lifecycle.HiltViewModel class * extends androidx.lifecycle.ViewModel

# Reproducible builds: prevent R8 non-deterministic inlining of appcompat methods
-keepclassmembers class androidx.appcompat.widget.ActionBarContextView {
    void setContentHeight(int);
}
-keepclassmembers class androidx.appcompat.app.AppCompatDelegateImpl$AppCompatWindowCallback {
    *;
}
