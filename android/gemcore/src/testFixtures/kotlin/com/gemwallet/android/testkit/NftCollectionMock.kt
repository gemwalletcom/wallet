package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import com.wallet.core.primitives.VerificationStatus

fun mockNftCollection(
    id: NFTCollectionId = mockNftCollectionId(),
) = NFTCollection(
    id = id,
    name = id.toIdentifier(),
    description = null,
    chain = id.chain,
    contractAddress = id.contractAddress,
    images = NFTImages(NFTResource("", "")),
    status = VerificationStatus.Verified,
    links = emptyList(),
)
