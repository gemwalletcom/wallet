// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Style

public struct VerificationStatusViewModel {
    public let status: VerificationStatus

    public init(status: VerificationStatus) {
        self.status = status
    }

    public var title: String {
        switch status {
        case .verified: .empty
        case .unverified: Localized.Asset.Verification.unverified
        case .suspicious: Localized.Asset.Verification.suspicious
        }
    }

    public var description: String {
        switch status {
        case .verified: String.empty
        case .unverified: Localized.Info.AssetStatus.Unverified.description
        case .suspicious: Localized.Info.AssetStatus.Suspicious.description
        }
    }

    public var statusStyle: TextStyle {
        switch status {
        case .verified: .calloutSecondary
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
    }

    public var assetImage: AssetImage {
        switch status {
        case .verified: AssetImage()
        case .unverified: AssetImage(placeholder: Images.TokenStatus.warning)
        case .suspicious: AssetImage(placeholder: Images.TokenStatus.risk)
        }
    }

    public var docsUrl: URL {
        AppUrl.docs(.tokenVerification)
    }
}
