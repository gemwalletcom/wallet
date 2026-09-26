package com.gemwallet.android.features.support.viewmodels

import android.content.Context
import com.gemwallet.android.application.device.cases.EnablePushForSupport
import com.gemwallet.android.data.services.store.queries.SupportMessagesQuery
import com.gemwallet.android.ui.R
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemPushState

@OptIn(ExperimentalCoroutinesApi::class)
class SupportChatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @Test
    fun `a failed push registration shows on the chat`() = runTest(testDispatcher) {
        val enablePushForSupport = mockk<EnablePushForSupport> {
            coEvery { enablePushForSupport() } returns GemPushState(isEnabled = true, result = GemPushResult.NotRegistered(GemErrorText.NetworkOffline))
        }
        val viewModel = SupportChatViewModel(
            supportService = mockk(relaxed = true),
            supportMessagesQuery = mockk<SupportMessagesQuery> { every { this@mockk() } returns flowOf(emptyList()) },
            getSupportTyping = mockk(relaxed = true),
            clearSupportTyping = mockk(relaxed = true),
            enablePushForSupport = enablePushForSupport,
            imageAttachmentFactory = mockk(relaxed = true),
            ioDispatcher = testDispatcher,
            context = mockk<Context> { every { getString(any()) } answers { "string:${firstArg<Int>()}" } },
        )

        advanceUntilIdle()

        assertEquals("string:${R.string.errors_network_offline}", viewModel.error.value)
    }
}
