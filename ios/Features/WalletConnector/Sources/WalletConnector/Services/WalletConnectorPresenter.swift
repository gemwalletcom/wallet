// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService

@Observable
public final class WalletConnectorPresenter: SigningRequestSheetPresentable, Sendable {
    public let sheets = SheetPresenter<WalletConnectorSheetType>()

    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false

    public init() {}

    public static func signMessageSheet(_ callback: SigningRequestCallback<SignMessagePayload>) -> WalletConnectorSheetType {
        .signMessage(callback)
    }

    public static func transferSheet(_ callback: SigningRequestCallback<SigningTransferData>) -> WalletConnectorSheetType {
        .transferData(callback)
    }
}
