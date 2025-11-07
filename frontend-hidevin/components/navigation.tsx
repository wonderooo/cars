import Link from "next/link"
import { Button } from "@/components/ui/button"
import { Car } from "lucide-react"
import { ThemeToggle } from "@/components/theme-toggle"

export function Navigation() {
    return (
        <nav className="fixed top-0 left-0 right-0 z-50 border-b border-border bg-background/80 backdrop-blur-lg">
            <div className="container mx-auto px-4 lg:px-8">
                <div className="flex h-16 items-center justify-between">
                    <Link href="/" className="flex items-center gap-2">
                        <Car className="h-6 w-6 text-accent" />
                        <span className="text-xl font-semibold text-foreground">VIN Clear</span>
                    </Link>

                    <div className="hidden md:flex items-center gap-8">
                        <Link href="#services" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            Services
                        </Link>
                        <Link
                            href="#how-it-works"
                            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                        >
                            How It Works
                        </Link>
                        <Link href="#pricing" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            Pricing
                        </Link>
                        <Link href="#faq" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            FAQ
                        </Link>
                    </div>

                    <div className="flex items-center gap-4">
                        <ThemeToggle />
                        <Button variant="ghost" size="sm" className="hidden sm:inline-flex">
                            Sign In
                        </Button>
                        <Button size="sm" variant={"default"}>
                            Get Started
                        </Button>
                    </div>
                </div>
            </div>
        </nav>
    )
}
