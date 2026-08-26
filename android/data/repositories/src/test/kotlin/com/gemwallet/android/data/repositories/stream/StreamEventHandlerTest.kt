package com.gemwallet.android.data.repositories.stream

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportTyping
import com.wallet.core.primitives.SupportTypingStatus
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemStreamService

class StreamEventHandlerTest {

    private val streamService = mockk<GemStreamService> {
        coEvery { handle(any(), any()) } returns Unit
    }
    private val sessionRepository = mockk<SessionRepository> {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }
    private val supportChatRepository = mockk<SupportChatRepository>(relaxed = true)
    private val handler = StreamEventHandler(streamService, sessionRepository, supportChatRepository)

    @Test
    fun `events are handled by core with the session currency`() = runTest {
        val text = """{"event":"priceAlerts","data":{"assets":[]}}"""

        handler.handle(text)

        coVerify { streamService.handle(text, Currency.USD.toJson()) }
        verify(exactly = 0) { supportChatRepository.updateTyping(any()) }
    }

    @Test
    fun `support typing event updates typing state`() = runTest {
        val typing = SupportTyping(status = SupportTypingStatus.On, agent = SupportAgent(name = "agent"))
        val text = """{"event":"support","data":{"type":"typing","data":${typing.toJson()}}}"""

        handler.handle(text)

        verify { supportChatRepository.updateTyping(typing) }
    }
}
