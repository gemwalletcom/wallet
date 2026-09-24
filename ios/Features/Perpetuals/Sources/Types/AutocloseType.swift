// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

public typealias AutocloseCompletion = (AutocloseSelection) -> Void

public enum AutocloseType {
    case modify(PerpetualPositionData, onTransferAction: TransferDataAction)
    case open(AutocloseOpenData, onComplete: AutocloseCompletion)
}
