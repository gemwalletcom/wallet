package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockNftCollectionId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.VerificationStatus

fun mockDbNftCollection(id: NFTCollectionId = mockNftCollectionId()) = DbNFTCollection(
    id = id,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
    status = VerificationStatus.Verified,
)
