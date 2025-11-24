/**
 * Simple test page to verify React and CSS are working
 */

function TestPage() {
  return (
    <div style={{ 
      padding: '2rem', 
      backgroundColor: '#111827', 
      minHeight: '100vh',
      color: '#ffffff'
    }}>
      <h1 style={{ fontSize: '2rem', marginBottom: '1rem' }}>
        Dashboard Test Page
      </h1>
      <p style={{ fontSize: '1rem', marginBottom: '1rem' }}>
        If you can see this with dark background, React is working!
      </p>
      <div style={{ 
        marginTop: '2rem', 
        padding: '1rem', 
        backgroundColor: '#1f2937', 
        borderRadius: '0.5rem',
        border: '1px solid #374151'
      }}>
        <p>Testing inline styles - this should work regardless of Tailwind</p>
        <p style={{ marginTop: '0.5rem', color: '#9ca3af' }}>
          If you see this, the issue is with Tailwind CSS classes
        </p>
      </div>
    </div>
  );
}

export default TestPage;
