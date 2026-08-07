// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct ConfirmTransferData: Sendable {
    let metadata: TransferDataMetadata
    let input: ConfirmTransferInput
    let simulation: ConfirmSimulationState
}
