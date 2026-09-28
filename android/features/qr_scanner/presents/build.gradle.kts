plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "com.gemwallet.android.features.qr_scanner.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))

    implementation(libs.camera.camera2)
    implementation(libs.camera.lifecycle)
    implementation(libs.camera.view)
    implementation(libs.compose.permissions)

    implementation(libs.ktx.core)
    implementation(libs.lifecycle.runtime)

    testImplementation(libs.junit)
}
