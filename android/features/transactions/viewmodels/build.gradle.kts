plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.transactions.viewmodels"
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":data:services:store"))
    api(project(":ui-models"))

    implementation(libs.lifecycle.viewmodel)
    implementation(libs.lifecycle.viewmodel.savedstate)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
    testImplementation(testFixtures(project(":gemcore")))
}
