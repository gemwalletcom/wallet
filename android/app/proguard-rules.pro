# Keep SourceFile stable and avoid path-derived R8 SourceFile names.
-keepattributes SourceFile
-renamesourcefileattribute SourceFile

# Allow optimization while preserving stack traces.
-optimizations !code/allocation/variable
-optimizations !class/unboxing/enum

-keepnames @dagger.hilt.android.lifecycle.HiltViewModel class * extends androidx.lifecycle.ViewModel

# Reproducible builds: prevent R8 non-deterministic inlining of appcompat methods
-keepclassmembers class androidx.appcompat.widget.ActionBarContextView {
    void setContentHeight(int);
}
-keepclassmembers class androidx.appcompat.app.AppCompatDelegateImpl$AppCompatWindowCallback {
    *;
}

# Room resolves its generated Database/Dao implementations by class name at runtime;
# keep them and the annotated declarations they're generated from intact.
-keep class * extends androidx.room.RoomDatabase { *; }
-keep @androidx.room.Dao class * { *; }
-keep @androidx.room.Entity class * { *; }
-keep class com.gemwallet.android.data.service.store.database.** { *; }
