// Puppeteer launch config for environments without userns/SUID sandbox
module.exports = {
  args: [
    '--no-sandbox',
    '--disable-setuid-sandbox'
  ]
};

