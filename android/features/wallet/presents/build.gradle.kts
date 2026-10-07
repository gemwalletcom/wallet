plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.wallet.presents"

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
        }
    }
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

    debugImplementation(libs.androidx.ui.test.manifest)
    testImplementation(libs.junit)
    testImplementation(libs.androidx.ui.test.junit4)
    testImplementation(testFixtures(project(":gemcore")))
}
