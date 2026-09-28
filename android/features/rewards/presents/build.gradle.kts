plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.rewards.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:rewards:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)

    debugImplementation(libs.androidx.ui.tooling)
    implementation(libs.androidx.ui.tooling.preview)
}
