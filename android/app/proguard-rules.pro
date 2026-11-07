# Keep SourceFile stable and avoid path-derived R8 SourceFile names.
-keepattributes SourceFile
-renamesourcefileattribute SourceFile

-keepnames @dagger.hilt.android.lifecycle.HiltViewModel class * extends androidx.lifecycle.ViewModel

# Reproducible builds: prevent R8 non-deterministic inlining of appcompat methods
-keepclassmembers class androidx.appcompat.widget.ActionBarContextView {
    void setContentHeight(int);
}
-keepclassmembers class androidx.appcompat.app.AppCompatDelegateImpl$AppCompatWindowCallback {
    *;
}

# Room only looks up <Database>_Impl by name, which Room's own rule keeps. These rules came
# back after a crash on update (#824) whose stack trace was not recorded. Each one changes
# the release DEX: prove a removal with an upgrade install of a release build, not by
# reading our sources.
-keep class * extends androidx.room.RoomDatabase { *; }
-keep @androidx.room.Dao class * { *; }
-keep @androidx.room.Entity class * { *; }
-keep class com.gemwallet.android.data.services.store.database.** { *; }
