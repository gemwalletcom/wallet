package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.testkit.mockDelegation
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.TimeUnit

class ValidatorItemTest {

    @Test
    fun availableInDuration_usesCompletionDateMillis() {
        val currentTimeMillis = 1_788_321_600_000L
        val durationMillis = TimeUnit.HOURS.toMillis(11) + TimeUnit.MINUTES.toMillis(6)
        val delegation = mockDelegation().let {
            it.copy(base = it.base.copy(completionDate = currentTimeMillis + durationMillis))
        }

        assertEquals(durationMillis, availableInDurationMillis(delegation, currentTimeMillis))
    }
}
