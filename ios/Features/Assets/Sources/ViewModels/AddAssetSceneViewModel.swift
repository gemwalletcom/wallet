// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import protocol Gemstone.GemExplorerServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AddAssetSceneViewModel {
    private let gatewayService: GatewayService
    private let explorerService: any GemExplorerServiceProtocol

    var state: StateViewType<AddAssetViewModel> = .noData
    var input: AddAssetInput

    var isPresentingScanner = false

    public init(wallet: Wallet, gatewayService: GatewayService, explorerService: any GemExplorerServiceProtocol) {
        self.gatewayService = gatewayService
        self.explorerService = explorerService
        input = AddAssetInput(chains: wallet.chainsWithTokens)
    }

    var title: String {
        Localized.Wallet.AddToken.title
    }

    var networkTitle: String {
        Localized.Transfer.network
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var actionButtonTitle: String {
        Localized.Wallet.Import.action
    }

    var addressTitleField: String {
        Localized.Wallet.Import.contractAddressField
    }

    var pasteImage: Image {
        Images.System.paste
    }

    var qrImage: Image {
        Images.System.qrCodeViewfinder
    }

    var errorSystemImage: String {
        SystemImage.errorOccurred
    }

    var chains: [Chain] {
        input.chains
    }

    var addressBinding: Binding<String> {
        Binding(
            get: { [self] in
                input.address ?? ""
            },
            set: { [self] in
                input.address = $0.isEmpty ? nil : $0
            },
        )
    }

    var warningImageStyle: ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: AssetImage(type: .emoji(Emoji.WalletAvatar.warning.rawValue)),
            imageSize: .image.semiMedium,
            alignment: .top,
            cornerRadiusType: .none,
        )
    }

    var tokenVerificationUrl: URL {
        AppUrl.docs(.tokenVerification)
    }

    var customTokenUrl: URL {
        AppUrl.docs(.addCustomToken)
    }
}

// MARK: - Business Logic

extension AddAssetSceneViewModel {
    func fetch() async {
        guard let chain = input.chain, let address = input.address, !address.isEmpty else {
            state = .noData
            return
        }

        state = .loading

        do {
            let asset = try await gatewayService.tokenData(chain: chain, tokenId: address)
            state = .data(AddAssetViewModel(asset: asset, explorerService: explorerService))
        } catch {
            state.setError(error)
        }
    }
}
