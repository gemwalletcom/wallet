import Components
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemReceiveAssetState
import struct Gemstone.GemReceiveNetworks
import protocol Gemstone.GemReceiveServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ReceiveSceneViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private(set) var assetState: GemReceiveAssetState
    private(set) var address: String

    var presentation: ReceivePresentationType?
    var renderedImage: UIImage?
    var isPresentingAlertMessage: AlertMessage?

    private let wallet: Wallet
    private let service: any GemReceiveServiceProtocol
    private let generator = QRCodeGenerator()
    let networks: GemReceiveNetworks
    private(set) var selectNetworkTask: Task<Void, Never>?

    private init(
        asset: Asset,
        associations: [AssetAssociation],
        wallet: Wallet,
        address: String,
        service: any GemReceiveServiceProtocol,
    ) {
        assetState = service.assetState(asset: asset.toGem())
        self.wallet = wallet
        self.address = address
        self.service = service
        networks = service.networks(
            asset: asset.toGem(),
            associations: associations.map(\.assetId.identifier),
            wallet: wallet.toGem(),
        )
    }

    public convenience init(assetData: AssetData, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetData.asset,
            associations: assetData.associations,
            wallet: wallet,
            address: assetData.account.address,
            service: service,
        )
    }

    public convenience init(assetAddress: AssetAddress, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetAddress.asset,
            associations: [],
            wallet: wallet,
            address: assetAddress.address,
            service: service,
        )
    }

    var title: String {
        Localized.Wallet.receive
    }

    var asset: Asset {
        assetState.asset.asset.toPrimitives()
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        assetState.warnings
            .map(\.text)
            .joined(separator: " ")
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: addressCopy(chain: asset.chain.toGem(), address: address))
    }

    var showNetworkSelector: Bool {
        networks.showsSelector
    }

    var networkSelectorModel: ReceiveNetworkSelectorViewModel {
        ReceiveNetworkSelectorViewModel(
            assetIds: networks.networks.map { AssetId(core: $0.assetId) },
        )
    }

    func chainModel(for assetId: AssetId) -> ChainViewModel {
        ChainViewModel(
            chain: assetId.chain,
            standard: networks.networks.first { $0.assetId == assetId.identifier }?.standard?.text,
        )
    }

    var isPresentingSheet: ReceivePresentationType? {
        get {
            switch presentation {
            case .share, .networkSelector: presentation
            case .copy, nil: nil
            }
        }
        set {
            presentation = newValue
        }
    }

    var isPresentingCopyToast: Bool {
        get {
            if case .copy = presentation {
                return true
            }
            return false
        }
        set {
            presentation = newValue ? .copy : nil
        }
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    private func selectNetwork(assetId: AssetId) async {
        do {
            let asset = try await service.asset(assetId: assetId.identifier).toPrimitives()
            let account = try wallet.account(for: asset.chain)
            try Task.checkCancellation()
            assetState = service.assetState(asset: asset.toGem())
            address = account.address
        } catch {
            guard !error.isCancelled else { return }
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    private func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize,
            ),
            logo: UIImage.name("logo-dark"),
        )
    }
}

// MARK: - Actions

extension ReceiveSceneViewModel {
    func onChangeAsset() async {
        do {
            try await service.enableAsset(walletId: wallet.id.id, assetId: asset.id.identifier)
        } catch {
            debugLog("ReceiveSceneViewModel enableAsset error: \(error)")
        }
    }

    func onSelectNetwork() {
        presentation = .networkSelector
    }

    func onFinishNetworkSelection(_ assetIds: [AssetId]) {
        presentation = nil
        guard let assetId = assetIds.first, assetId != asset.id else { return }

        selectNetworkTask?.cancel()
        selectNetworkTask = Task { await selectNetwork(assetId: assetId) }
    }

    func onShareSheet() {
        presentation = .share
    }

    func onCopyAddress() {
        presentation = .copy
    }

    func onLoadImage() async {
        renderedImage = nil
        renderedImage = await generateQRCode()
    }
}
