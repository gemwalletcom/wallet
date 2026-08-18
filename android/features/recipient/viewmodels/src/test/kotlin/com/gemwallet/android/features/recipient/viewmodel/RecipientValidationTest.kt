package com.gemwallet.android.features.recipient.viewmodel

import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockNameRecord
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
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
        val record = mockNameRecord(name = accountId, chain = chain, address = accountId, provider = NameProvider.Near)
        val validAddress = addressValidator(true)
        val invalidAddress = addressValidator(false)

        assertTrue(destination.isValidRecipient(accountId, chain, record, validAddress))
        assertFalse(destination.isValidRecipient(otherAccountId, chain, record, validAddress))
        assertFalse(destination.isValidRecipient(accountId, Chain.Ethereum, record, validAddress))
        assertFalse(destination.isValidRecipient(accountId, chain, record, invalidAddress))
        assertFalse(destination.isValidRecipient(accountId, chain, null, invalidAddress))
        assertTrue(destination.isValidRecipient(accountId, chain, null, validAddress))

        val ethereumName = "example.eth"
        val ethereumAddress = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"
        val ethereumDestination = DestinationAddress(address = ethereumAddress, name = ethereumName)
        val ethereumRecord = mockNameRecord(
            name = ethereumName,
            address = ethereumAddress,
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

    private fun addressValidator(result: Boolean): (String, Chain) -> Boolean = { _, _ -> result }
}
