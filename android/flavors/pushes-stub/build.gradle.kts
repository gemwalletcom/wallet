plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "com.gemwallet.android.flovers.stub"
}

dependencies {
    implementation(project(":gemcore"))
}
