"use client"

import type React from "react"
import {useState} from "react"
import {Button} from "@/components/ui/button"
import {Input} from "@/components/ui/input"
import {Card} from "@/components/ui/card"
import {Search, Shield, Zap} from "lucide-react"
import {redirect} from "next/navigation";

export function HeroSection() {
    const [vin, setVin] = useState("")

    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault()
        redirect(`/vin/${vin}`)
    }

    return (
        <section className="relative pt-32 pb-20 px-4 lg:px-8">
            <div className="container mx-auto">
                <div className="max-w-4xl mx-auto text-center">
                    {/* Announcement Badge */}
                    <div
                        className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-accent/10 border border-accent/20 mb-8">
                        <div className="h-2 w-2 rounded-full bg-accent animate-pulse"/>
                        <span className="text-sm text-muted-foreground">Trusted by 10,000+ dealers nationwide</span>
                    </div>

                    {/* Main Heading */}
                    <h1 className="text-5xl md:text-6xl lg:text-7xl font-bold text-foreground mb-6 text-balance leading-tight">
                        Clear Your Vehicle History from Copart & IAAI
                    </h1>

                    {/* Subheading */}
                    <p className="text-lg md:text-xl text-muted-foreground mb-12 max-w-2xl mx-auto text-pretty leading-relaxed">
                        Professional vehicle history deletion service. Remove your car records from auction databases
                        quickly,
                        securely, and permanently.
                    </p>

                    {/* VIN Search Form */}
                    <Card className="p-6 md:p-8 bg-card border-border max-w-2xl mx-auto mb-12">
                        <form onSubmit={handleSearch} className="space-y-4">
                            <div className="flex flex-col sm:flex-row gap-3">
                                <div className="relative flex-1">
                                    <Search
                                        className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground"/>
                                    <Input
                                        type="text"
                                        placeholder="Enter your 17-digit VIN number"
                                        value={vin}
                                        onChange={(e) => setVin(e.target.value)}
                                        className="pl-10 h-12 bg-background border-border text-foreground"
                                        maxLength={17}
                                    />
                                </div>
                                <Button
                                    type="submit"
                                    size="lg"
                                    variant="default"
                                    className="h-12 px-8"
                                >
                                    Check VIN
                                </Button>
                            </div>
                            <p className="text-xs text-muted-foreground text-left">
                                {"We'll check if your vehicle appears in Copart or IAAI databases"}
                            </p>
                        </form>
                    </Card>

                    {/* Feature Pills */}
                    <div className="flex flex-wrap justify-center gap-4">
                        <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-card border border-border">
                            <Shield className="h-4 w-4 text-accent"/>
                            <span className="text-sm text-foreground">100% Secure</span>
                        </div>
                        <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-card border border-border">
                            <Zap className="h-4 w-4 text-accent"/>
                            <span className="text-sm text-foreground">Fast Processing</span>
                        </div>
                        <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-card border border-border">
                            <svg className="h-4 w-4 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth={2}
                                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                            </svg>
                            <span className="text-sm text-foreground">Guaranteed Results</span>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    )
}
