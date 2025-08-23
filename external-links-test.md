# External Links Test

This test file verifies that all links in the markdown content open in the system's default browser.

## Test Links

### External HTTP/HTTPS Links
These should open in the system browser:
- [GitHub](https://github.com) - Main GitHub website
- [Google](https://www.google.com) - Google search engine  
- [Example HTTP](http://example.com) - HTTP link test
- [Hugging Face](https://huggingface.co) - ML model hub

### Email Links
This should open the default email client:
- [Contact Us](mailto:test@example.com) - Email link test

### Anchor Links
These should work normally within the application (not open externally):
- [Link to top](#external-links-test) - Internal anchor link
- [Link to test section](#test-links) - Another internal link

### FTP Links
This should open in the system browser or appropriate application:
- [FTP Example](ftp://ftp.example.com) - FTP link test

## Expected Behavior

✅ **External links (http/https)** should open in the system's default browser, not within the Oxide-Lab application

✅ **Email links (mailto)** should open the system's default email client

✅ **Anchor links (#)** should work normally within the application for page navigation

✅ **FTP links** should open in the appropriate system application

## Test Instructions

1. Open this content in the Oxide-Lab chat interface or model description
2. Click on each type of link above
3. Verify that external links open in your system browser
4. Verify that anchor links work within the application
5. Verify that email links open your email client

If all links behave as expected, the external links feature is working correctly!