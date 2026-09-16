package com.gemwallet.android.features.settings.contacts.viewmodels

import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemContactAddressField
import org.junit.Assert.assertEquals
import org.junit.Test

class ContactAddressInputTest {

    @Test
    fun memoIsOfferedOnlyForChainsThatSupportIt() {
        assertEquals(listOf(GemContactAddressField.NETWORK, GemContactAddressField.ADDRESS), ContactAddressInput(chain = Chain.Ethereum).fields)
        assertEquals(
            listOf(GemContactAddressField.NETWORK, GemContactAddressField.ADDRESS, GemContactAddressField.MEMO),
            ContactAddressInput(chain = Chain.Cosmos).fields,
        )
    }
}
