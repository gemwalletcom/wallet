plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.contacts.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:qr_scanner:presents"))
    implementation(project(":features:contacts:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)
}
