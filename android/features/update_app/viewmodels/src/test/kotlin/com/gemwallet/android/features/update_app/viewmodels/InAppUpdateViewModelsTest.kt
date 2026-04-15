package com.gemwallet.android.features.update_app.viewmodels

import com.gemwallet.android.cases.device.RequestPushToken
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.AppUpdateInfo
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class InAppUpdateViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val skippedVersion = MutableStateFlow("")
    private val latestAppUpdate = MutableStateFlow<AppUpdateInfo?>(null)
    private val userConfig = mockk<UserConfig>(relaxed = true)
    private val requestPushToken = mockk<RequestPushToken>(relaxed = true)
    private lateinit var updateService: FakeInAppUpdateService

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { userConfig.getAppVersionSkip() } returns skippedVersion
        every { userConfig.getLatestAppUpdate() } returns latestAppUpdate
        updateService = FakeInAppUpdateService()
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `required update remains available even when skipped`() = runTest(testDispatcher) {
        skippedVersion.value = "2.0.0"
        latestAppUpdate.value = AppUpdateInfo(version = "2.0.0", isRequired = true)

        val viewModel = createViewModel()
        advanceUntilIdle()

        val update = viewModel.updateAvailable.value
        assertNotNull(update)
        assertEquals("2.0.0", update?.version)
        assertTrue(update?.isRequired == true)
    }

    @Test
    fun `skip ignores required update`() = runTest(testDispatcher) {
        latestAppUpdate.value = AppUpdateInfo(version = "2.0.0", isRequired = true)

        val viewModel = createViewModel()
        advanceUntilIdle()

        viewModel.skip()
        advanceUntilIdle()

        coVerify(exactly = 0) { userConfig.setAppVersionSkip(any()) }
    }

    @Test
    fun `update does not launch overlapping downloads and cancel marks canceled`() = runTest(testDispatcher) {
        latestAppUpdate.value = AppUpdateInfo(version = "2.0.0", isRequired = false)

        val viewModel = createViewModel()
        advanceUntilIdle()

        viewModel.update()
        advanceUntilIdle()

        viewModel.update()
        advanceUntilIdle()

        assertEquals(1, updateService.downloadCalls)

        viewModel.cancel()
        advanceUntilIdle()

        assertEquals(1, updateService.cancelCalls)
        assertTrue(viewModel.downloadState.value == DownloadState.Canceled)
    }

    @Test
    fun `update does nothing when no update exists`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        viewModel.update()
        advanceUntilIdle()

        assertEquals(0, updateService.downloadCalls)
    }

    private fun createViewModel() = InAppUpdateViewModel(
        buildInfo = BuildInfo(
            platformStore = PlatformStore.ApkUniversal,
            versionName = "1.0.0",
            versionCode = 1,
            requestPushToken = requestPushToken,
        ),
        userConfig = userConfig,
        updateService = updateService,
    )

    private class FakeInAppUpdateService : InAppUpdateService {
        var downloadCalls = 0
        var cancelCalls = 0

        override fun canRequestPackageInstalls(): Boolean = true

        override suspend fun clearDownloadedUpdate() = Unit

        override suspend fun download(version: String, onProgress: (Float?) -> Unit) {
            downloadCalls += 1
            kotlinx.coroutines.awaitCancellation()
        }

        override fun installDownloadedUpdate(version: String) = Unit

        override fun cancel() {
            cancelCalls += 1
        }
    }
}
