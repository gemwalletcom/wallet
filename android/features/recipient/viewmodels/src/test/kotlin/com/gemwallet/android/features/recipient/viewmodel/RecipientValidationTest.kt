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
    fun validRecipientRequiresValidAddressAndMatchingRecord() {
        val accountId = "wrap.near"
        val otherAccountId = "other.near"
        val chain = Chain.Near
        val destination = DestinationAddress(address = accountId, name = accountId)
        val record = NameRecord(name = accountId, chain = chain, address = accountId, provider = NameProvider.Near)
        val validAddress = addressValidator(true)
        val invalidAddress = addressValidator(false)

        assertTrue(destination.isValidRecipient(accountId, chain, record, validAddress))
        assertFalse(destination.isValidRecipient(otherAccountId, chain, record, validAddress))
        assertFalse(destination.isValidRecipient(accountId, Chain.Ethereum, record, validAddress))
        assertFalse(destination.isValidRecipient(accountId, chain, record, invalidAddress))
        assertFalse(destination.isValidRecipient(accountId, chain, null, invalidAddress))
        assertTrue(destination.isValidRecipient(accountId, chain, null, validAddress))

        val ethereumName = "example.eth"
        val ethereumAddress = "0x1234567890123456789012345678901234567890"
        val ethereumDestination = DestinationAddress(address = ethereumAddress, name = ethereumName)
        val ethereumRecord = NameRecord(
            name = ethereumName,
            chain = Chain.Ethereum,
            address = ethereumAddress,
            provider = NameProvider.Ens,
        )
        val unresolvedEthereumRecord = ethereumRecord.copy(address = ethereumName)
        val unresolvedEthereumDestination = DestinationAddress(address = ethereumName, name = ethereumName)

        assertTrue(ethereumDestination.isValidRecipient(ethereumName, Chain.Ethereum, ethereumRecord, validAddress))
        assertFalse(
            unresolvedEthereumDestination.isValidRecipient(
                ethereumName,
                Chain.Ethereum,
                unresolvedEthereumRecord,
                invalidAddress,
            ),
        )
    }

    private fun addressValidator(result: Boolean) = object : ValidateAddressOperator {
        override fun invoke(address: String, chain: Chain): Result<Boolean> = Result.success(result)
    }
}
