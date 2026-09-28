plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.settings.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:qr_scanner:presents"))
    implementation(project(":features:settings:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)

    debugImplementation(libs.androidx.ui.tooling)
    implementation(libs.androidx.ui.tooling.preview)
}
