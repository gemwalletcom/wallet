# Disable R8 optimization to keep pg-map-id and DEX output stable across clean builds.
-dontoptimize
-keep,allowshrinking,allowoptimization class * { <methods>; }
-keep class com.gemwallet.android.** { *; }
