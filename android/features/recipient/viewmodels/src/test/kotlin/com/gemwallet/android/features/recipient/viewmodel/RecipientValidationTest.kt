package com.gemwallet.android.features.recipient.viewmodel

import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RecipientValidationTest {

    @Test
    fun resolvedNameRequiresMatchingRecord() {
        val accountId = "wrap.near"
        val otherAccountId = "other.near"
        val chain = Chain.Near
        val rejectAddress = addressValidator(false)
        val destination = DestinationAddress(address = accountId, name = accountId)
        val record = NameRecord(name = accountId, chain = chain, address = accountId, provider = NameProvider.Near)

        assertTrue(destination.isValidRecipient(accountId, chain, record, rejectAddress))
        assertFalse(destination.isValidRecipient(otherAccountId, chain, record, rejectAddress))
        assertFalse(destination.isValidRecipient(accountId, Chain.Ethereum, record, rejectAddress))
        assertFalse(destination.isValidRecipient(accountId, chain, null, rejectAddress))
        assertTrue(destination.isValidRecipient(accountId, chain, null, addressValidator(true)))
    }

    private fun addressValidator(result: Boolean) = object : ValidateAddressOperator {
        override fun invoke(address: String, chain: Chain): Result<Boolean> = Result.success(result)
    }
}
