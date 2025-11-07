import { Navigation } from "@/components/navigation"
import { HeroSection } from "@/components/hero-section"
import { ServicesSection } from "@/components/services-section"
import { HowItWorksSection } from "@/components/how-it-works-section"
import { Footer } from "@/components/footer"

export default function HomePage() {
    return (
        <main className="min-h-screen">
            <Navigation />
            <HeroSection />
            <ServicesSection />
            <HowItWorksSection />
            <Footer />
        </main>
    )
}
