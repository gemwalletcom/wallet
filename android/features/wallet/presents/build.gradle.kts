plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.wallet.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:wallet:viewmodels"))
    implementation(project(":features:assets:presents"))
    implementation(project(":features:assets:viewmodels"))
    implementation(project(":features:perpetuals:presents"))
    implementation(project(":features:nft:presents"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)

    debugImplementation(libs.androidx.ui.tooling)
    implementation(libs.androidx.ui.tooling.preview)

    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
}
