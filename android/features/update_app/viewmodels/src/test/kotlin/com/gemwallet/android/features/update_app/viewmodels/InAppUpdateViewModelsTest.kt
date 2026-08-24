package com.gemwallet.android.features.update_app.viewmodels

import com.gemwallet.android.application.update.coordinators.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.coordinators.SkipAppUpdate
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class InAppUpdateViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val offer = MutableStateFlow<AppUpdateOffer?>(null)
    private lateinit var skipAppUpdate: FakeSkipAppUpdate
    private lateinit var updateService: FakeInAppUpdateService

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        skipAppUpdate = FakeSkipAppUpdate()
        updateService = FakeInAppUpdateService()
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `in app apk offer is available`() = runTest(testDispatcher) {
        offer.value = AppUpdateOffer("2.0.0", isRequired = true, channel = AppUpdateChannel.InAppApk)

        val viewModel = createViewModel()
        advanceUntilIdle()

        val update = viewModel.updateAvailable.value
        assertNotNull(update)
        assertEquals("2.0.0", update?.version)
        assertTrue(update?.isRequired == true)
    }

    @Test
    fun `store offer is not shown as an in app update`() = runTest(testDispatcher) {
        offer.value = AppUpdateOffer("2.0.0", isRequired = false, channel = AppUpdateChannel.Store)

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertNull(viewModel.updateAvailable.value)
    }

    @Test
    fun `skip ignores required update`() = runTest(testDispatcher) {
        offer.value = AppUpdateOffer("2.0.0", isRequired = true, channel = AppUpdateChannel.InAppApk)

        val viewModel = createViewModel()
        advanceUntilIdle()

        viewModel.skip()
        advanceUntilIdle()

        assertTrue(skipAppUpdate.skippedVersions.isEmpty())
    }

    @Test
    fun `skip stores the optional update version`() = runTest(testDispatcher) {
        offer.value = AppUpdateOffer("2.0.0", isRequired = false, channel = AppUpdateChannel.InAppApk)

        val viewModel = createViewModel()
        advanceUntilIdle()

        viewModel.skip()
        advanceUntilIdle()

        assertEquals(listOf("2.0.0"), skipAppUpdate.skippedVersions)
    }

    @Test
    fun `update does not launch overlapping downloads and cancel marks canceled`() = runTest(testDispatcher) {
        offer.value = AppUpdateOffer("2.0.0", isRequired = false, channel = AppUpdateChannel.InAppApk)

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
        observeAppUpdateOffer = object : ObserveAppUpdateOffer {
            override fun observeAppUpdateOffer(): Flow<AppUpdateOffer?> = offer
        },
        skipAppUpdate = skipAppUpdate,
        updateService = updateService,
    )

    private class FakeSkipAppUpdate : SkipAppUpdate {
        val skippedVersions = mutableListOf<String>()

        override suspend fun skipAppUpdate(version: String) {
            skippedVersions.add(version)
        }
    }

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
