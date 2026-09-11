package com.gemwallet.android.features.settings.contacts.viewmodels

import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ContactAddressInputTest {

    @Test
    fun memoIsOfferedOnlyForChainsThatSupportIt() {
        assertFalse(ContactAddressInput(chain = Chain.Ethereum).showMemo)
        assertTrue(ContactAddressInput(chain = Chain.Cosmos).showMemo)
    }
}
