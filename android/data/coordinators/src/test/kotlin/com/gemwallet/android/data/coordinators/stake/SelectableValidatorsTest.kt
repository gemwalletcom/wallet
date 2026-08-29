package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class SelectableValidatorsTest {

    @Test
    fun selectableValidators_filtersOutUnnamedValidators() {
        val active = mockDelegationValidator(chain = Chain.Bitcoin, id = "active", name = "Active", apr = 10.0)
        val unnamed = mockDelegationValidator(chain = Chain.Bitcoin, name = "")
        val inactive = mockDelegationValidator(chain = Chain.Bitcoin, id = "inactive", isActive = false, apr = 20.0)

        val result = selectableValidators(
            listOf(active, unnamed, inactive),
        )

        assertEquals(listOf(active.id), result.map { it.id })
    }

    @Test
    fun selectableValidators_sortsByAprDescending() {
        val lowApr = mockDelegationValidator(chain = Chain.Bitcoin, id = "low", name = "Low", apr = 4.0)
        val highApr = mockDelegationValidator(chain = Chain.Bitcoin, id = "high", name = "High", apr = 12.0)

        val result = selectableValidators(
            listOf(lowApr, highApr),
        )

        assertEquals(listOf(highApr.id, lowApr.id), result.map { it.id })
    }
}
