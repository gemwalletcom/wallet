package com.gemwallet.android.features.settings.contacts.viewmodels

import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.ui.models.name.NameRecordState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ContactAddressInputTest {

    private val chain = Chain.Ethereum
    private val resolvedAddress = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"

    private fun completed(name: String = "vitalik.eth") = NameRecordState.Complete(
        NameRecord(name = name, chain = chain, address = resolvedAddress, provider = NameProvider.Ens),
    )

    @Test
    fun confirmStaysDisabledWhileResolving() {
        val input = ContactAddressInput(chain = chain, address = "vitalik.eth", isAddressValid = true)

        assertFalse(input.copy(nameResolveState = NameRecordState.Loading).isConfirmEnabled)
        assertTrue(input.copy(nameResolveState = completed()).isConfirmEnabled)
    }

    @Test
    fun memoIsOfferedOnlyForChainsThatSupportIt() {
        assertFalse(ContactAddressInput(chain = Chain.Ethereum).showMemo)
        assertTrue(ContactAddressInput(chain = Chain.Cosmos).showMemo)
    }
}
