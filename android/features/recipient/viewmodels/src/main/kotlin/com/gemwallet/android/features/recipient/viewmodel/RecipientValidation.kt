package com.gemwallet.android.features.recipient.viewmodel

import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.ext.matchesRecipient
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

internal fun DestinationAddress.isValidRecipient(
    inputAddress: String,
    chain: Chain,
    nameRecord: NameRecord?,
    validateAddress: ValidateAddressOperator,
): Boolean = validateAddress(address, chain).getOrNull() == true &&
    (nameRecord == null || nameRecord.matchesRecipient(inputAddress, address, chain))
