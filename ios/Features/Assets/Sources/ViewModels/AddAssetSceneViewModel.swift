// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAddAssetServiceProtocol
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
    private let service: any GemAddAssetServiceProtocol
    private let wallet: Wallet

    var state: StateViewType<AddAssetViewModel> = .noData
    var input: AddAssetInput

    var isPresentingScanner = false
    var loadTrigger: AddAssetLoadTrigger?

    public init(wallet: Wallet, service: any GemAddAssetServiceProtocol) {
        self.service = service
        self.wallet = wallet
        let chains = service.chains(wallet: wallet)
        input = AddAssetInput(chains: chains, chain: service.defaultChain(chains: chains))
    }

    var title: String {
        Localized.Wallet.AddToken.title
    }

    var networksModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(state: .data(.plain(input.chains)))
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
    func setInput(_ address: String) {
        input.address = address
        setLoadTrigger(isImmediate: true)
    }

    func onChangeAddress() {
        guard loadTrigger?.address != input.address else { return }
        setLoadTrigger(isImmediate: false)
    }

    func onSubmitAddress() {
        setLoadTrigger(isImmediate: true)
    }

    func load() async {
        guard let trigger = loadTrigger else { return }

        state = .loading

        do {
            let asset = try await service.token(chain: trigger.chain, address: trigger.address)
            state = .data(AddAssetViewModel(asset: asset, link: service.tokenUrl(chain: asset.chain, tokenId: asset.tokenId ?? "")))
        } catch {
            state.setError(error)
        }
    }

    func add(_ asset: Asset) {
        Task {
            do {
                try await service.add(wallet: wallet, assetId: asset.id)
            } catch {
                debugLog("AddAssetSceneViewModel add asset error: \(error)")
            }
        }
    }

    private func setLoadTrigger(isImmediate: Bool) {
        guard let chain = input.chain, let address = input.address, !address.isEmpty else {
            state = .noData
            loadTrigger = nil
            return
        }
        loadTrigger = AddAssetLoadTrigger(chain: chain, address: address, isImmediate: isImmediate)
    }
}
