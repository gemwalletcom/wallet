// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct StateHeroView: View {
    private let systemImage: String
    private let title: String
    private let description: String

    public init(systemImage: String, title: String, description: String) {
        self.systemImage = systemImage
        self.title = title
        self.description = description
    }

    public var body: some View {
        VStack(spacing: .medium) {
            Image(systemName: systemImage)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .symbolRenderingMode(.palette)
                .foregroundStyle(Colors.blue, Colors.blue.opacity(.opacity38))
                .frame(size: .image.large)
            VStack(spacing: .small) {
                Text(title)
                    .textStyle(.boldTitle)
                Text(description)
                    .textStyle(.bodySecondary)
            }
            .multilineTextAlignment(.center)
            .minimumScaleFactor(0.85)
        }
    }
}
