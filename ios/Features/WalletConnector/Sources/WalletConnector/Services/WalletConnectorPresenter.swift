// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SigningRequestService

@Observable
public final class WalletConnectorPresenter: Sendable {
    public let sheets = SheetPresenter<WalletConnectorSheetType>()

    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false

    public init() {}
}
