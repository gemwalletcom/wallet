package com.gemwallet.android.features.confirm.presents.components

import com.gemwallet.android.features.confirm.models.ConfirmProperty
import org.junit.Assert.assertEquals
import org.junit.Test

class PropertyDestinationTest {
    @Test
    fun stakeDestination_displaysValidatorName() {
        val model = ConfirmProperty.Destination.Stake("Allnodes.com")

        assertEquals("Allnodes.com", model.displayData())
    }

    @Test
    fun providerDestination_displaysProviderName() {
        val model = ConfirmProperty.Destination.Provider("1inch")

        assertEquals("1inch", model.displayData())
    }

    @Test
    fun transferDestination_displaysDomainWhenAvailable() {
        val model = ConfirmProperty.Destination.Transfer(
            domain = "vitalik.eth",
            address = "0x1234567890abcdef1234567890abcdef12345678",
        )

        assertEquals("vitalik.eth", model.displayData())
    }
}
