// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Style
import SwiftUI

public typealias InfoSheetAction = @MainActor @Sendable () -> Void

public enum InfoSheetButton: Sendable {
    case url(URL, title: String? = nil)
    case action(title: String, action: InfoSheetAction)

    var title: String {
        switch self {
        case let .url(_, title): title ?? Localized.Common.learnMore
        case let .action(title, _): title
        }
    }
}

public enum InfoSheetImage: Sendable {
    case image(Image)
    case assetImage(AssetImage)
}

public struct InfoSheetModel: Sendable {
    public let title: String
    public let description: String
    public let image: InfoSheetImage?
    public let button: InfoSheetButton?
    public let secondaryButtons: [InfoSheetButton]
    public let titleStyle: TextStyle
    public let descriptionStyle: TextStyle

    public init(
        title: String,
        description: String,
        image: InfoSheetImage? = nil,
        button: InfoSheetButton? = nil,
        secondaryButtons: [InfoSheetButton] = [],
        titleStyle: TextStyle = .boldTitle,
        descriptionStyle: TextStyle = .bodySecondary,
    ) {
        self.title = title
        self.description = description
        self.image = image
        self.button = button
        self.secondaryButtons = secondaryButtons
        self.titleStyle = titleStyle
        self.descriptionStyle = descriptionStyle
    }

    var shouldShowButton: Bool {
        button != nil || !secondaryButtons.isEmpty
    }
}
