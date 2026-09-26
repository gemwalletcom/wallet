package com.gemwallet.android.features.contacts.viewmodels

import com.gemwallet.android.features.contacts.viewmodels.models.ContactAddressForm
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemContactAddressField

class ContactEditorViewModelTest {

    @Test
    fun memoIsOfferedOnlyForChainsThatSupportIt() {
        assertFalse(GemContactAddressField.MEMO in ContactAddressForm(chain = Chain.Ethereum).fields)
        assertTrue(GemContactAddressField.MEMO in ContactAddressForm(chain = Chain.Cosmos).fields)
    }
}
