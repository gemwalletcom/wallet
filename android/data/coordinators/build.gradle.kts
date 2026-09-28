plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.data.coordinators"

    testOptions {
        unitTests {
            isReturnDefaultValues = true
        }
    }
}

dependencies {
    implementation(project(":data:services:gemstone"))
    implementation(project(":data:services:store"))

    compileOnly(libs.compose.runtime.annotation)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.ktx.core)
    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(testFixtures(project(":data:services:store")))
}
