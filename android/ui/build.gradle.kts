plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "com.gemwallet.android.ui"

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
        }
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
}

dependencies {
    api(project(":ui-models"))

    // Compose
    api(libs.androidx.material3.adaptive.android)
    api(libs.compose.ui)
    api(libs.compose.material3)
    api(libs.kotlinx.collections.immutable)
    api(libs.compose.activity)
    api(libs.browser)

    // QRCode scanner: only for none private data: recipient, memo, amount, etc
    // QR Code
    api(libs.zxing.core)

    // Images
    api(libs.coil.compose)
    api(libs.coil.network)
    api(libs.coil.svg)

    // Permissions request
    api(libs.compose.permissions)

    implementation(libs.ktx.core)
    implementation(libs.lifecycle.runtime)
    implementation(libs.lifecycle.viewmodel.compose)
    implementation(libs.navigation3.runtime)
    implementation(libs.material)

    debugImplementation(libs.androidx.ui.tooling)
    debugImplementation(libs.androidx.ui.test.manifest)
    implementation(libs.androidx.ui.tooling.preview)

    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.mockk.android)
    testImplementation(libs.androidx.ui.test.junit4)
}
