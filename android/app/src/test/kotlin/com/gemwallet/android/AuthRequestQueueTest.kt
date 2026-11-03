package com.gemwallet.android

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import kotlin.time.Duration.Companion.minutes
import kotlin.time.TestTimeSource

class AuthRequestQueueTest {

    @Test
    fun enqueue_startsFirstRequestAndQueuesNextRequests() {
        val queue = AuthRequestQueue()

        val first = queue.enqueue {}
        val second = queue.enqueue {}

        assertEquals(0L, first?.id)
        assertNull(second)
    }

    @Test
    fun completeActive_allowsNextQueuedRequestToStart() {
        val queue = AuthRequestQueue()

        queue.enqueue {}
        queue.enqueue {}

        val completed = queue.completeActive()
        val next = queue.startNext()

        assertEquals(0L, completed?.id)
        assertEquals(1L, next?.id)
    }

    @Test
    fun completeActive_withDifferentRequestId_keepsActiveRequest() {
        val queue = AuthRequestQueue()

        queue.enqueue {}
        queue.enqueue {}

        assertNull(queue.completeActive(1L))

        val completed = queue.completeActive(0L)
        val next = queue.startNext()

        assertEquals(0L, completed?.id)
        assertEquals(1L, next?.id)
    }

    @Test
    fun chainedRequest_doesNotSkipAlreadyQueuedRequest() {
        val queue = AuthRequestQueue()

        queue.enqueue {}
        queue.enqueue {}
        queue.completeActive()

        val chained = queue.enqueue {}
        val next = queue.completeActive()

        assertEquals(1L, chained?.id)
        assertEquals(1L, next?.id)
    }

    @Test
    fun activeTimeLeft_countsFromWhenTheRequestStarts() {
        val time = TestTimeSource()
        val queue = AuthRequestQueue(timeout = 5.minutes, timeSource = time)

        queue.enqueue {}
        queue.enqueue {}
        time += 4.minutes
        assertEquals(1.minutes, queue.activeTimeLeft())

        queue.completeActive()
        assertNull(queue.activeTimeLeft())

        queue.startNext()
        time += 2.minutes
        assertEquals(3.minutes, queue.activeTimeLeft())
    }
}
