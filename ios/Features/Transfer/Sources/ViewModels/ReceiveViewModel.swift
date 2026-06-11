import BalanceService
import Components
import Foundation
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ReceiveViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    let assetModel: AssetViewModel
    let wallet: Wallet
    let address: String
    let assetsEnabler: any AssetsEnabler
    let balanceService: BalanceService
    let generator = QRCodeGenerator()

    public var isPresentingShareSheet: Bool = false
    public var isPresentingCopyToast: Bool = false
    public var isPresentingInfoSheet: InfoSheetType?
    public var renderedImage: UIImage?

    public init(
        assetModel: AssetViewModel,
        wallet: Wallet,
        address: String,
        assetsEnabler: any AssetsEnabler,
        balanceService: BalanceService,
    ) {
        self.assetModel = assetModel
        self.wallet = wallet
        self.address = address
        self.assetsEnabler = assetsEnabler
        self.balanceService = balanceService
    }

    private var feeAsset: Asset {
        assetModel.asset.chain.asset
    }

    var title: String {
        Localized.Receive.title("")
    }

    var addressShort: String {
        AddressFormatter(style: .short, address: address, chain: assetModel.asset.chain).value()
    }

    var shareTitle: String {
        Localized.Common.share
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        [Localized.Receive.warning(assetModel.symbol.boldMarkdown(), assetModel.networkFullName.boldMarkdown()), memoWarningText]
            .compactMap(\.self)
            .joined(separator: " ")
    }

    private var memoWarningText: String? {
        switch assetModel.asset.chain {
        case .xrp where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noDestinationTagRequired
        case _ where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noMemoRequired
        default: nil
        }
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .address(assetModel.asset, address: addressShort),
            copyValue: address,
        )
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    func enableAsset() async {
        do {
            try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetModel.asset.id], enabled: true)
        } catch {
            debugLog("ReceiveViewModel enableAsset error: \(error)")
        }
    }

    func presentFeeWarningIfNeeded() {
        guard assetModel.asset.id.type == .token, feeAsset.chain != .hyperCore else { return }
        guard let balance = try? balanceService.getBalance(walletId: wallet.id, assetId: feeAsset.id), balance.available == .zero else { return }
        isPresentingInfoSheet = .receiveNetworkFee(
            asset: assetModel.asset,
            feeAsset: feeAsset,
            image: AssetViewModel(asset: feeAsset).assetImage,
        )
    }

    func generateQRCode() async -> UIImage? {
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

extension ReceiveViewModel {
    func onTaskOnce() {
        presentFeeWarningIfNeeded()
        Task {
            await enableAsset()
        }
    }

    func onShareSheet() {
        isPresentingShareSheet = true
    }

    func onCopyAddress() {
        isPresentingCopyToast = true
    }

    func onLoadImage() async {
        renderedImage = await generateQRCode()
    }
}
