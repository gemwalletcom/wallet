// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemContractCallData {
    func map() -> ContractCallData {
        ContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval?.map(),
            gasLimit: gasLimit,
        )
    }
}

public extension ContractCallData {
    func map() -> GemContractCallData {
        GemContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval?.map(),
            gasLimit: gasLimit,
        )
    }
}
