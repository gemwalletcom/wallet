plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}
android {
    namespace = "com.gemwallet.android.features.rewards.viewmodels"
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":data:services:store"))
    api(project(":ui-models"))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.viewmodel)
    implementation(libs.lifecycle.viewmodel.savedstate)

    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.mockk.android)
    testImplementation(libs.kotlinx.coroutines.test)
}
