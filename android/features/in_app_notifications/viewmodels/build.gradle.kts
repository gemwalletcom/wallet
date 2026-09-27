plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.in_app_notifications.viewmodels"
}

dependencies {
    implementation(project(":gemcore"))
    implementation(project(":data:services:store"))
    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.viewmodel)
}
