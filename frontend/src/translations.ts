import { TranslationDict } from "./types";

export const translations: Record<"en" | "fr", TranslationDict> = {
  en: {
    navProducts: "How it works",
    navDocuments: "Architecture",
    navIllustrations: "Proof",
    navPartners: "Contracts",
    navContact: "Contact",
    badge: "Zero-Knowledge Trade Finance on Stellar",
    titleMain: "Bill of",
    titleSub: "Zero",
    desc: "Privacy-preserving Letter-of-Credit settlement. A zero-knowledge proof attests that a private invoice and bill of lading satisfy the LC's terms; a Soroban contract verifies it on Stellar and releases escrowed stablecoin — without ever revealing the documents.",
    btnRegister: "Prove LC Compliance",
    btnGetStarted: "Deploy Escrow",
    statClients: "LC Rules Proven",
    statCountries: "Byte Journal",
    statWebsites: "Documents Private",
    statAttacks: "ZK Primitives",
    terminalTitle: "INITIALIZING ZK PROVER",
    terminalSub: "Load a private document set and generate a Groth16 proof that it satisfies the Letter-of-Credit terms. Nothing about the documents leaves your machine.",
    liveFeedTitle: "Live Settlement Stream",
    liveFeedDesc: "Real-time proof generation, on-chain verification, and escrow releases across Letters of Credit.",
    registerSuccess: "Proof generated. Seal and 80-byte journal are ready for on-chain verification.",
    provisionTitle: "Escrow Deployment Control",
    provisionDesc: "Configure and deploy the Letter-of-Credit escrow contract to a Stellar network."
  },
  fr: {
    navProducts: "Fonctionnement",
    navDocuments: "Architecture",
    navIllustrations: "Preuve",
    navPartners: "Contrats",
    navContact: "Contact",
    badge: "Finance commerciale à divulgation nulle sur Stellar",
    titleMain: "Bill of",
    titleSub: "Zero",
    desc: "Règlement confidentiel de lettres de crédit. Une preuve à divulgation nulle atteste qu'une facture et un connaissement privés respectent les termes de la LC ; un contrat Soroban la vérifie sur Stellar et libère le stablecoin sous séquestre — sans jamais révéler les documents.",
    btnRegister: "Prouver la conformité",
    btnGetStarted: "Déployer le séquestre",
    statClients: "Règles LC prouvées",
    statCountries: "Journal (octets)",
    statWebsites: "Documents privés",
    statAttacks: "Primitives ZK",
    terminalTitle: "INITIALISATION DU PROVER ZK",
    terminalSub: "Chargez un jeu de documents privés et générez une preuve Groth16 attestant le respect des termes de la lettre de crédit. Aucune donnée des documents ne quitte votre machine.",
    liveFeedTitle: "Flux de règlement en direct",
    liveFeedDesc: "Génération de preuves, vérification on-chain et libérations de séquestre en temps réel sur les lettres de crédit.",
    registerSuccess: "Preuve générée. Le sceau et le journal de 80 octets sont prêts pour la vérification on-chain.",
    provisionTitle: "Contrôle de déploiement du séquestre",
    provisionDesc: "Configurez et déployez le contrat de séquestre de la lettre de crédit sur un réseau Stellar."
  }
};

// Settlement-pipeline event types shown in the live stream.
export const threatTypes = [
  { type: "release" as const, label: "Escrow Released to Seller", status: "FUNDS SETTLED", color: "text-emerald-400" },
  { type: "proof" as const, label: "Groth16 Proof Generated", status: "SEAL 73c457ba", color: "text-[#BDF589]" },
  { type: "verify" as const, label: "On-chain Proof Verified", status: "VERIFIER OK", color: "text-[#636EB4]" },
  { type: "fund" as const, label: "Escrow Funded", status: "FUNDS LOCKED", color: "text-amber-400" },
  { type: "reject" as const, label: "Non-compliant Docs Rejected", status: "GUEST PANIC", color: "text-rose-400" },
];

// Letter-of-Credit / document references that scroll through the stream.
export const targetClusters = [
  "LC-1001", "LC-1042-ACME", "BoL-SH-2207", "INV-99031",
  "LC-1188", "LC-2025-EU", "BoL-TK-0412"
];

// Trade-corridor country codes.
export const countriesList = ["SG", "DE", "JP", "US", "GB", "NL", "CN", "IN", "BR", "CH"];
