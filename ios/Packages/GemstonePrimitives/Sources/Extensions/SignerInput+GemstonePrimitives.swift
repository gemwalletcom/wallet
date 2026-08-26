// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension SignerInput {
    func map() throws -> GemSignerInput {
        try GemSignerInput(
            input: GemTransactionLoadInput.map(signerInput: self),
            fee: fee.map(),
        )
    }
}
