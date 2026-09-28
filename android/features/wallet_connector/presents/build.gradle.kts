plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.wallet_connector.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:qr_scanner:presents"))
    implementation(project(":features:wallet_connector:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)
}
