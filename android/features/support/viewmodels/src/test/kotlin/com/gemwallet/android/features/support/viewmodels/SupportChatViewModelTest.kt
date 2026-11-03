package com.gemwallet.android.features.support.viewmodels

import android.content.Context
import android.net.Uri
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import com.gemwallet.android.application.device.cases.EnablePushForSupport
import com.gemwallet.android.application.support.cases.SendSupportMessage
import com.gemwallet.android.data.services.store.queries.SupportMessagesQuery
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ui.R
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemPushState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSupportServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SupportChatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    @Test
    fun `a failed push registration shows on the chat`() = runTest(testDispatcher) {
        val enablePushForSupport = mockk<EnablePushForSupport> {
            coEvery { enablePushForSupport() } returns GemPushState(isEnabled = true, result = GemPushResult.NotRegistered(GemErrorText.NetworkOffline))
        }
        val viewModel = viewModel(supportService = mockk(relaxed = true), enablePushForSupport = enablePushForSupport)

        advanceUntilIdle()

        assertEquals("string:${R.string.errors_network_offline}", viewModel.error.value)
    }

    @Test
    fun `a failed sync with no messages shows the error instead of the empty chat`() = runTest(testDispatcher) {
        val supportService = mockk<GemSupportServiceInterface>(relaxed = true) {
            coEvery { refresh(any()) } returns GemLoadState.Error(GemServiceException.Gateway("offline"))
        }
        val viewModel = viewModel(supportService = supportService, enablePushForSupport = mockk { coEvery { enablePushForSupport() } returns null })
        advanceUntilIdle()
        assertTrue(viewModel.phase.value is GemListPhase.Empty)

        viewModel.load()
        advanceUntilIdle()

        assertEquals("offline", ((viewModel.phase.value as GemListPhase.Error).error as GemServiceException.Gateway).msg)
    }

    @Test
    fun `sending a message sends it trimmed and clears the draft`() = runTest(testDispatcher) {
        val sendSupportMessage = mockk<SendSupportMessage>(relaxed = true)
        val viewModel = viewModel(supportService = mockk(relaxed = true), enablePushForSupport = mockk { coEvery { enablePushForSupport() } returns null }, sendSupportMessage = sendSupportMessage)
        viewModel.messageState.setTextAndPlaceCursorAtEnd("  My swap is stuck ")

        viewModel.sendMessage()
        advanceUntilIdle()

        coVerify(exactly = 1) { sendSupportMessage.sendText("My swap is stuck") }
        assertEquals("", viewModel.messageState.text.toString())
    }

    @Test
    fun `an unreadable photo is reported and the readable ones are sent`() = runTest(testDispatcher) {
        val readable = mockk<Uri>()
        val unreadable = mockk<Uri>()
        val image = byteArrayOf(1)
        val sendSupportMessage = mockk<SendSupportMessage>(relaxed = true)
        val imageAttachmentFactory = mockk<SupportImageAttachmentFactory> {
            coEvery { fromUri(readable) } returns image
            coEvery { fromUri(unreadable) } returns null
        }
        val viewModel = viewModel(
            supportService = mockk(relaxed = true),
            enablePushForSupport = mockk { coEvery { enablePushForSupport() } returns null },
            sendSupportMessage = sendSupportMessage,
            imageAttachmentFactory = imageAttachmentFactory,
        )

        viewModel.sendImages(listOf(unreadable, readable))
        advanceUntilIdle()

        coVerify(exactly = 1) { sendSupportMessage.sendImages(listOf(image)) }
        assertEquals("string:${R.string.errors_not_supported}", viewModel.error.value)
    }

    private fun viewModel(
        supportService: GemSupportServiceInterface,
        enablePushForSupport: EnablePushForSupport,
        sendSupportMessage: SendSupportMessage = mockk(relaxed = true),
        imageAttachmentFactory: SupportImageAttachmentFactory = mockk(relaxed = true),
    ) = SupportChatViewModel(
        supportService = supportService,
        sendSupportMessage = sendSupportMessage,
        supportMessagesQuery = mockk<SupportMessagesQuery> { every { this@mockk() } returns flowOf(emptyList()) },
        getSupportTyping = mockk(relaxed = true),
        clearSupportTyping = mockk(relaxed = true),
        enablePushForSupport = enablePushForSupport,
        imageAttachmentFactory = imageAttachmentFactory,
        ioDispatcher = testDispatcher,
        context = mockk<Context> { every { getString(any()) } answers { "string:${firstArg<Int>()}" } },
    )
}
