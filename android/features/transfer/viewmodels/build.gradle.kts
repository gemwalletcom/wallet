plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.transfer.viewmodels"

    testOptions {
        unitTests {
            isReturnDefaultValues = true
        }
    }
}

dependencies {
    api(project(":ui-models"))
    implementation(project(":ui"))
    implementation(project(":gemcore"))
    implementation(project(":data:services:store"))

    implementation(libs.lifecycle.viewmodel)
    implementation(libs.lifecycle.viewmodel.savedstate)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    testImplementation(libs.junit)
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
}
