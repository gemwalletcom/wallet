package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.SupportMessagesQuery
import com.gemwallet.android.testkit.mockSupportMessage
import com.gemwallet.android.testkit.mockSupportMessageImage
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SupportMessagesQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: SupportMessagesQuery

    private val agentReply = mockSupportMessage(
        id = "agent-reply",
        content = "Hello",
        sender = SupportMessageSender.Agent(SupportAgent(name = "Alice")),
        status = SupportMessageStatus.Sent,
        createdAt = 20,
        images = listOf(mockSupportMessageImage(id = "image", url = "https://gem/image.png", width = 10, height = 20)),
    )
    private val first = mockSupportMessage(id = "first", content = "Hi", status = SupportMessageStatus.Sent, createdAt = 10)
    private val sameTimeB = mockSupportMessage(id = "same-b", content = "B", status = SupportMessageStatus.Failed, createdAt = 30)
    private val sameTimeA = mockSupportMessage(id = "same-a", content = "A", status = SupportMessageStatus.Sending, createdAt = 30)

    @Before
    fun setUp() {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = SupportMessagesQuery(database.supportMessagesDao())
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsEveryMessageOldestFirstThenById() = runBlocking(Dispatchers.IO) {
        database.supportMessagesDao().addMessages(listOf(sameTimeB, agentReply, sameTimeA, first).map { it.toRecord() })

        assertEquals(listOf(first, agentReply, sameTimeA, sameTimeB), query().first())
    }

    @Test
    fun emptyChatListsNoMessages() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<SupportMessage>(), query().first())
    }
}
